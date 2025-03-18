class KlipBin < Formula
  version '0.1.0'
  desc 'Copy/paste anything over the network'
  homepage 'https://github.com/lmaotrigine/klip'
  if OS.mac?
    url = "https://github.com/lmaotrigine/klip/releases/download/v#{version}/klip-x86_64-apple-darwin.tar.xz"
    sha256 "somesha"
  elsif OS.linux?
    url = "https://github.com/lmaotrigine/klip/releases/download/v#{version}/klip-x86_64-unknown-linux-musl.tar.xz"
    sha256 "somesha"
  end
  conflicts_with "klip"
  def install
    bin.install "klip"
    man1.install "doc/klip.1"
    bash_completion.install "complete/klip.bash"
    zsh_completion.install "complete/_klip"
  end
end
