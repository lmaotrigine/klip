complete -c klip -n "__fish_use_subcommand" -s c -l config -d 'path to the configuration file (default=$HOME/.klip.toml)' -r -F
complete -c klip -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c klip -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c klip -n "__fish_use_subcommand" -f -a "copy" -d 'store content'
complete -c klip -n "__fish_use_subcommand" -f -a "paste" -d 'retrieve content'
complete -c klip -n "__fish_use_subcommand" -f -a "move" -d 'retrieve and delete content'
complete -c klip -n "__fish_use_subcommand" -f -a "serve" -d 'start a server'
complete -c klip -n "__fish_use_subcommand" -f -a "genkeys" -d 'generate keys'
complete -c klip -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c klip -n "__fish_seen_subcommand_from copy" -s h -l help -d 'Print help'
complete -c klip -n "__fish_seen_subcommand_from paste" -s h -l help -d 'Print help'
complete -c klip -n "__fish_seen_subcommand_from move" -s h -l help -d 'Print help'
complete -c klip -n "__fish_seen_subcommand_from serve" -l max-clients -d 'the maximum number of simultaneous client connections' -r
complete -c klip -n "__fish_seen_subcommand_from serve" -l max-len-mb -d 'maximum content length to accept in MiB (0=unlimited)' -r
complete -c klip -n "__fish_seen_subcommand_from serve" -s t -l timeout -d 'connection timeout (in seconds)' -r
complete -c klip -n "__fish_seen_subcommand_from serve" -s d -l data-timeout -d 'data transmission timeout (in seconds)' -r
complete -c klip -n "__fish_seen_subcommand_from serve" -s h -l help -d 'Print help'
complete -c klip -n "__fish_seen_subcommand_from serve" -s V -l version -d 'Print version'
complete -c klip -n "__fish_seen_subcommand_from genkeys" -s p -l password -d 'derive the keys from a password (default=random keys)'
complete -c klip -n "__fish_seen_subcommand_from genkeys" -s h -l help -d 'Print help'
complete -c klip -n "__fish_seen_subcommand_from genkeys" -s V -l version -d 'Print version'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "copy" -d 'store content'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "paste" -d 'retrieve content'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "move" -d 'retrieve and delete content'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "serve" -d 'start a server'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "genkeys" -d 'generate keys'
complete -c klip -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from copy; and not __fish_seen_subcommand_from paste; and not __fish_seen_subcommand_from move; and not __fish_seen_subcommand_from serve; and not __fish_seen_subcommand_from genkeys; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
