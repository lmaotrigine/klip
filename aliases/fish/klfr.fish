function klfr --description 'send a whole directory to the klip clipboard, as a tar archive'
  tar cJpvf - $argv | klip copy;
end
