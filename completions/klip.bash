#shellcheck disable=SC2207
_klip() {
  local i cur prev opts cmd
  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  cmd=""
  opts=""

  for i in "${COMP_WORDS[@]}"; do
    case "${cmd},${i}" in
      ",$1")
        cmd="klip"
        ;;
      klip,copy)
        cmd="klip__copy"
        ;;
      klip,genkeys)
        cmd="klip__genkeys"
        ;;
      klip,help)
        cmd="klip__help"
        ;;
      klip,move)
        cmd="klip__move"
        ;;
      klip,paste)
        cmd="klip__paste"
        ;;
      klip,serve)
        cmd="klip__serve"
        ;;
      klip,version)
        cmd="klip__version"
        ;;
      klip__help,copy)
        cmd="klip__help__copy"
        ;;
      klip__help,genkeys)
        cmd="klip__help__genkeys"
        ;;
      klip__help,help)
        cmd="klip__help__help"
        ;;
      klip__help,move)
        cmd="klip__help__move"
        ;;
      klip__help,paste)
        cmd="klip__help__paste"
        ;;
      klip__help,serve)
        cmd="klip__help__serve"
        ;;
      klip__help,version)
        cmd="klip__help__version"
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
    klip__copy)
      opts="-h --help"
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
    klip__genkeys)
      opts="-p -h -V --password --help --version"
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
    klip__help)
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
    klip__help__copy)
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
    klip__help__genkeys)
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
    klip__help__help)
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
    klip__help__move)
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
    klip__help__paste)
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
    klip__help__serve)
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
    klip__help__version)
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
    klip__move)
      opts="-h --help"
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
    klip__paste)
      opts="-h --help"
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
    klip__serve)
      opts="-t -d -h -V --max-clients --max-len-mb --timeout --data-timeout --help --version"
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
    klip__version)
      opts="-h --help"
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
