
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
            cand -c 'path to the configuration file (default=$HOME/.klip.toml)'
            cand --config 'path to the configuration file (default=$HOME/.klip.toml)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand copy 'store content'
            cand paste 'retrieve content'
            cand move 'retrieve and delete content'
            cand serve 'start a server'
            cand genkeys 'generate keys'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'klip;copy'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'klip;paste'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'klip;move'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'klip;serve'= {
            cand --max-clients 'the maximum number of simultaneous client connections'
            cand --max-len-mb 'maximum content length to accept in MiB (0=unlimited)'
            cand -t 'connection timeout (in seconds)'
            cand --timeout 'connection timeout (in seconds)'
            cand -d 'data transmission timeout (in seconds)'
            cand --data-timeout 'data transmission timeout (in seconds)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'klip;genkeys'= {
            cand -p 'derive the keys from a password (default=random keys)'
            cand --password 'derive the keys from a password (default=random keys)'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'klip;help'= {
            cand copy 'store content'
            cand paste 'retrieve content'
            cand move 'retrieve and delete content'
            cand serve 'start a server'
            cand genkeys 'generate keys'
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
        &'klip;help;help'= {
        }
    ]
    $completions[$command]
}
