#!/usr/bin/env python3

from __future__ import annotations

import json
import logging
import os
import re
import subprocess
import sys
from pathlib import Path
from urllib.request import Request, urlopen

_log = logging.getLogger(__name__)

_workflows_dir = Path(__file__).parent.with_name('.github') / 'workflows'
_re = re.compile(r'uses:\s+(?P<repo>[\w-]+/[\w-]+)@(?:(?P<commit>[a-f0-9]{40})\s*#\s*)?(?P<version>.+)$', re.MULTILINE)


def try_token() -> str | None:
    if env := os.getenv('GITHUB_TOKEN'):
        return env
    try:
        return subprocess.check_output(['gh', 'auth', 'token'], encoding='utf-8').strip()
    except subprocess.CalledProcessError:
        return None


def get_pinned_commit(repo: str, version: str) -> str:
    url = f'https://api.github.com/repos/{repo}/commits/{version}'
    headers = {'Accept': 'application/vnd.github.v3+json', 'User-Agent': 'actions/0.0.1'}
    if token := try_token():
        headers['Authorization'] = f'token {token}'
    req = Request(url, headers=headers)
    resp = urlopen(req)
    data = resp.read().decode('utf-8')
    return json.loads(data)['sha']


def update_actions() -> None:
    for workflow in _workflows_dir.glob('*.yml'):
        text = workflow.read_text()
        orig_text = text
        seen: dict[tuple[str, str], str] = {}
        for m in _re.finditer(text):
            repo, version, prev_commit = m.group('repo'), m.group('version'), m.group('commit')
            try:
                commit = seen[repo, version]
            except KeyError:
                _log.debug('%s: checking %s@%s for latest commit...', workflow.name, repo, version)
                seen[repo, version] = commit = get_pinned_commit(repo, version)
            if commit == prev_commit:
                _log.info('%s: %s@%s is already pinned to latest commit, skipping...', workflow.name, repo, version)
                continue
            _log.info('%s: updating %s@%s from %s to %s', workflow.name, repo, version, prev_commit, commit)
            text = text.replace(m.group(0), f'uses: {repo}@{commit} # {version}')
        if text == orig_text:
            _log.info('%s: no updates found, skipping...', workflow.name)
            continue
        workflow.write_text(text)


if __name__ == '__main__':
    status = subprocess.check_output(['git', 'status', '--untracked-files=no', '--porcelain'], encoding='utf-8')
    if status.strip():
        _log.error('git working directory is not clean, please commit or stash changes before running this script')
        sys.exit(1)
    logging.basicConfig(level=logging.INFO, format='%(levelname)s: %(message)s')
    update_actions()
