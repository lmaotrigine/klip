alias klc='klip copy'
alias klm='klip move'
alias klp='klip paste'
alias klpr='klip paste | tar xJpvf -'
alias klz='klip copy < /dev/null'

klf() {
  klip copy < $1
}

klfr() {
  tar cJpvf - ${1:-.} | klip copy
}

klo() {
  echo "$*" | klip copy
}
