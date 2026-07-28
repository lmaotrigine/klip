using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'klip' -ScriptBlock {
  param($wordToComplete, $commandAst, $cursorPosition)

  $commandElements = $commandAst.CommandElements
  $command = @(
    'klip'
    for ($i = 1; $i -lt $commandElements.Count; $i++) {
      $element = $commandElements[$i]
      if (
        $element -isnot [StringConstantExpressionAst] -or
        $element.StringConstantType -ne [StringConstantType]::BareWord -or
        $element.Value.StartsWith('-') -or
        $element.Value -eq $wordToComplete
      ) {
        break
      }
      $element.Value
    }) -join ';'

  $completions = @(switch ($command) {
    'klip' {
      [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Path to the configuration file (default=$HOME/.klip.toml)')
      [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Path to the configuration file (default=$HOME/.klip.toml)')
      [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
      [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
      [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
      [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
      [CompletionResult]::new('copy', 'copy', [CompletionResultType]::ParameterValue, 'Store content')
      [CompletionResult]::new('paste', 'paste', [CompletionResultType]::ParameterValue, 'Retrieve content')
      [CompletionResult]::new('move', 'move', [CompletionResultType]::ParameterValue, 'Retrieve and delete content')
      [CompletionResult]::new('serve', 'serve', [CompletionResultType]::ParameterValue, 'Start a server')
      [CompletionResult]::new('genkeys', 'genkeys', [CompletionResultType]::ParameterValue, 'Generate keys')
      [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print version')
      [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
      break
    }
    'klip;copy' {
      break
    }
    'klip;paste' {
      break
    }
    'klip;move' {
      break
    }
    'klip;serve' {
      [CompletionResult]::new('--max-clients', '--max-clients', [CompletionResultType]::ParameterName, 'Maximum number of simultaneous client connections')
      [CompletionResult]::new('--max-len-mb', '--max-len-mb', [CompletionResultType]::ParameterName, 'Maximum content length to accept in MiB (0=unlimited)')
      [CompletionResult]::new('-t', '-t', [CompletionResultType]::ParameterName, 'Connection timeout (in seconds)')
      [CompletionResult]::new('--timeout', '--timeout', [CompletionResultType]::ParameterName, 'Connection timeout (in seconds)')
      [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Data transmission timeout (in seconds)')
      [CompletionResult]::new('--data-timeout', '--data-timeout', [CompletionResultType]::ParameterName, 'Data transmission timeout (in seconds)')
      break
    }
    'klip;genkeys' {
      [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'Derive the keys from a password (default=random keys)')
      [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'Derive the keys from a password (default=random keys)')
      break
    }
    'klip;version' {
      break
    }
    'klip;help' {
      [CompletionResult]::new('copy', 'copy', [CompletionResultType]::ParameterValue, 'Store content')
      [CompletionResult]::new('paste', 'paste', [CompletionResultType]::ParameterValue, 'Retrieve content')
      [CompletionResult]::new('move', 'move', [CompletionResultType]::ParameterValue, 'Retrieve and delete content')
      [CompletionResult]::new('serve', 'serve', [CompletionResultType]::ParameterValue, 'Start a server')
      [CompletionResult]::new('genkeys', 'genkeys', [CompletionResultType]::ParameterValue, 'Generate keys')
      [CompletionResult]::new('version', 'version', [CompletionResultType]::ParameterValue, 'Print version')
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
