
use builtin;
use str;

set edit:completion:arg-completer[klip] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'klip'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'klip'= {
            cand -c 'Path to the configuration file (default=$HOME/.klip.toml)'
            cand --config 'Path to the configuration file (default=$HOME/.klip.toml)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand copy 'Store content'
            cand paste 'Retrieve content'
            cand move 'Retrieve and delete content'
            cand serve 'Start a server'
            cand genkeys 'Generate keys'
            cand version 'Print version'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'klip;copy'= {
        }
        &'klip;paste'= {
        }
        &'klip;move'= {
        }
        &'klip;serve'= {
            cand --max-clients 'Maximum number of simultaneous client connections'
            cand --max-len-mb 'Maximum content length to accept in MiB (0=unlimited)'
            cand -t 'Connection timeout (in seconds)'
            cand --timeout 'Connection timeout (in seconds)'
            cand -d 'Data transmission timeout (in seconds)'
            cand --data-timeout 'Data transmission timeout (in seconds)'
        }
        &'klip;genkeys'= {
            cand -p 'Derive the keys from a password (default=random keys)'
            cand --password 'Derive the keys from a password (default=random keys)'
        }
        &'klip;version'= {
        }
        &'klip;help'= {
            cand copy 'Store content'
            cand paste 'Retrieve content'
            cand move 'Retrieve and delete content'
            cand serve 'Start a server'
            cand genkeys 'Generate keys'
            cand version 'Print version'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'klip;help;copy'= {
        }
        &'klip;help;paste'= {
        }
        &'klip;help;move'= {
        }
        &'klip;help;serve'= {
        }
        &'klip;help;genkeys'= {
        }
        &'klip;help;version'= {
        }
        &'klip;help;help'= {
        }
    ]
    $completions[$command]
}
