class KlipBin < Formula
  version '0.1.0'
  desc 'Copy/paste anything over the network'
  homepage 'https://git.5ht2.me/lmaotrigine/klip'
  if OS.mac?
    url = "https://git.5ht2.me/lmaotrigine/klip/-/releases/v#{version}/downloads/klip-x86_64-apple-darwin.tar.xz"
    sha256 "somesha"
  elsif OS.linux?
    url = "https://git.5ht2.me/lmaotrigine/klip/-/releases/v#{version}/downloads/klip-x86_64-unknown-linux-musl.tar.xz"
    sha256 "somesha"
  end
  conflicts_with "klip"
  def install
    bin.install "klip"
    #man1.install "doc/klip.1"
    #bash_completion.install "complete/klip.bash"
    #zsh_completion.install "complete/_klip"
  end
end
