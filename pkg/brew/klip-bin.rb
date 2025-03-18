# typed: strict
# frozen_string_literal: true

# Homebrew formula for klip using binaries from GitHub releases
class KlipBin < Formula
  desc "Copy/paste anything over the network"
  homepage "https://github.com/lmaotrigine/klip"
  version "0.1.0"
  if OS.mac?
    url "https://github.com/lmaotrigine/klip/releases/download/#{version}/klip-x86_64-apple-darwin.full.zip"
    sha256 "80c4824aad513a2735eb4cb91b42950bf01fa613dcb2ee24b7012660a0523669"
  elsif OS.linux?
    url "https://github.com/lmaotrigine/klip/releases/download/#{version}/klip-x86_64-unknown-linux-musl.full.tar.xz"
    sha256 "c976f3f33745057318bb6260614e43c70a6f8b8698b069fa0eaa863cc2dbb40e"
  end
  def install
    bin.install "klip"
    man1.install "doc/klip.1"
    bash_completion.install "completions/klip.bash"
    zsh_completion.install "completions/_klip"
  end
end
