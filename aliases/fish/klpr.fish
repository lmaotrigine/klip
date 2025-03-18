function klpr --description 'extract klip clipboard content sent using the klfr command'
  klip paste | tar xJpvf -;
end
