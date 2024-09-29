# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_klip_global_optspecs
	string join \n c/config= h/help V/version
end

function __fish_klip_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_klip_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_klip_using_subcommand
	set -l cmd (__fish_klip_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c klip -n "__fish_klip_needs_command" -s c -l config -d 'path to the configuration file (default=$HOME/.klip.toml)' -r -F
complete -c klip -n "__fish_klip_needs_command" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_needs_command" -s V -l version -d 'Print version'
complete -c klip -n "__fish_klip_needs_command" -f -a "copy" -d 'store content'
complete -c klip -n "__fish_klip_needs_command" -f -a "paste" -d 'retrieve content'
complete -c klip -n "__fish_klip_needs_command" -f -a "move" -d 'retrieve and delete content'
complete -c klip -n "__fish_klip_needs_command" -f -a "serve" -d 'start a server'
complete -c klip -n "__fish_klip_needs_command" -f -a "genkeys" -d 'generate keys'
complete -c klip -n "__fish_klip_needs_command" -f -a "version" -d 'show version information'
complete -c klip -n "__fish_klip_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c klip -n "__fish_klip_using_subcommand copy" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand paste" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand move" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand serve" -l max-clients -d 'the maximum number of simultaneous client connections' -r
complete -c klip -n "__fish_klip_using_subcommand serve" -l max-len-mb -d 'maximum content length to accept in MiB (0=unlimited)' -r
complete -c klip -n "__fish_klip_using_subcommand serve" -s t -l timeout -d 'connection timeout (in seconds)' -r
complete -c klip -n "__fish_klip_using_subcommand serve" -s d -l data-timeout -d 'data transmission timeout (in seconds)' -r
complete -c klip -n "__fish_klip_using_subcommand serve" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand serve" -s V -l version -d 'Print version'
complete -c klip -n "__fish_klip_using_subcommand genkeys" -s p -l password -d 'derive the keys from a password (default=random keys)'
complete -c klip -n "__fish_klip_using_subcommand genkeys" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand genkeys" -s V -l version -d 'Print version'
complete -c klip -n "__fish_klip_using_subcommand version" -s h -l help -d 'Print help'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "copy" -d 'store content'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "paste" -d 'retrieve content'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "move" -d 'retrieve and delete content'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "serve" -d 'start a server'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "genkeys" -d 'generate keys'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "version" -d 'show version information'
complete -c klip -n "__fish_klip_using_subcommand help; and not __fish_seen_subcommand_from copy paste move serve genkeys version help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
