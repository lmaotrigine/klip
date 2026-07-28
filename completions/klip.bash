#shellcheck disable=SC2207
_klip() {
  local i cur prev opts cmd
  COMPREPLY=()
  if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
    cur="$2"
  else
    cur="${COMP_WORDS[COMP_CWORD]}"
  fi
  prev="$3"
  cmd=""
  opts=""

  for i in "${COMP_WORDS[@]:0:COMP_CWORD}"; do
    case "${cmd},${i}" in
      ",$1")
        cmd="klip"
        ;;
      klip,copy)
        cmd="klip__subcmd__copy"
        ;;
      klip,genkeys)
        cmd="klip__subcmd__genkeys"
        ;;
      klip,help)
        cmd="klip__subcmd__help"
        ;;
      klip,move)
        cmd="klip__subcmd__move"
        ;;
      klip,paste)
        cmd="klip__subcmd__paste"
        ;;
      klip,serve)
        cmd="klip__subcmd__serve"
        ;;
      klip,version)
        cmd="klip__subcmd__version"
        ;;
      klip__subcmd__help,copy)
        cmd="klip__subcmd__help__subcmd__copy"
        ;;
      klip__subcmd__help,genkeys)
        cmd="klip__subcmd__help__subcmd__genkeys"
        ;;
      klip__subcmd__help,help)
        cmd="klip__subcmd__help__subcmd__help"
        ;;
      klip__subcmd__help,move)
        cmd="klip__subcmd__help__subcmd__move"
        ;;
      klip__subcmd__help,paste)
        cmd="klip__subcmd__help__subcmd__paste"
        ;;
      klip__subcmd__help,serve)
        cmd="klip__subcmd__help__subcmd__serve"
        ;;
      klip__subcmd__help,version)
        cmd="klip__subcmd__help__subcmd__version"
        ;;
      *)
        ;;
    esac
  done

  case "${cmd}" in
    klip)
      opts="-c -h -V --config --help --version copy paste move serve genkeys version help"
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        --config)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        -c)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__copy)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__genkeys)
      opts="-p --password"
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help)
      opts="copy paste move serve genkeys version help"
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__copy)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__genkeys)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__help)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__move)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__paste)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__serve)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__help__subcmd__version)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__move)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
          COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
          return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__paste)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__serve)
      opts="-t -d --max-clients --max-len-mb --timeout --data-timeout"
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
          COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
          return 0
      fi
      case "${prev}" in
        --max-clients)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        --max-len-mb)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        --timeout)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        -t)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        --data-timeout)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        -d)
          COMPREPLY=($(compgen -f "${cur}"))
          return 0
          ;;
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
    klip__subcmd__version)
      opts=""
      if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
        COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
        return 0
      fi
      case "${prev}" in
        *)
          COMPREPLY=()
          ;;
      esac
      COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
      return 0
      ;;
  esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
  complete -F _klip -o nosort -o bashdefault -o default klip
else
  complete -F _klip -o bashdefault -o default klip
fi
