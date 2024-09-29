
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'klip' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'klip'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'klip' {
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'path to the configuration file (default=$HOME/.klip.toml)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'path to the configuration file (default=$HOME/.klip.toml)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('copy', 'copy', [CompletionResultType]::ParameterValue, 'store content')
            [CompletionResult]::new('paste', 'paste', [CompletionResultType]::ParameterValue, 'retrieve content')
            [CompletionResult]::new('move', 'move', [CompletionResultType]::ParameterValue, 'retrieve and delete content')
            [CompletionResult]::new('serve', 'serve', [CompletionResultType]::ParameterValue, 'start a server')
            [CompletionResult]::new('genkeys', 'genkeys', [CompletionResultType]::ParameterValue, 'generate keys')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'show version information')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'klip;copy' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'klip;paste' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'klip;move' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'klip;serve' {
            [CompletionResult]::new('--max-clients', '--max-clients', [CompletionResultType]::ParameterName, 'the maximum number of simultaneous client connections')
            [CompletionResult]::new('--max-len-mb', '--max-len-mb', [CompletionResultType]::ParameterName, 'maximum content length to accept in MiB (0=unlimited)')
            [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'connection timeout (in seconds)')
            [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'connection timeout (in seconds)')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'data transmission timeout (in seconds)')
            [CompletionResult]::new('--data-timeout', '--data-timeout', [CompletionResultType]::ParameterName, 'data transmission timeout (in seconds)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'klip;genkeys' {
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'derive the keys from a password (default=random keys)')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'derive the keys from a password (default=random keys)')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'klip;version' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'klip;help' {
            [CompletionResult]::new('copy', 'copy', [CompletionResultType]::ParameterValue, 'store content')
            [CompletionResult]::new('paste', 'paste', [CompletionResultType]::ParameterValue, 'retrieve content')
            [CompletionResult]::new('move', 'move', [CompletionResultType]::ParameterValue, 'retrieve and delete content')
            [CompletionResult]::new('serve', 'serve', [CompletionResultType]::ParameterValue, 'start a server')
            [CompletionResult]::new('genkeys', 'genkeys', [CompletionResultType]::ParameterValue, 'generate keys')
            [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'show version information')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'klip;help;copy' {
            break
        }
        'klip;help;paste' {
            break
        }
        'klip;help;move' {
            break
        }
        'klip;help;serve' {
            break
        }
        'klip;help;genkeys' {
            break
        }
        'klip;help;version' {
            break
        }
        'klip;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
