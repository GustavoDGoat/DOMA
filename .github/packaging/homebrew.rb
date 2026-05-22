# typed: false
# frozen_string_literal: true

class Doma < Formula
  desc "Terminal-based general-purpose AI chat interface with 90s nuclear aesthetic"
  homepage "https://github.com/GustavoDGoat/DOMA"
  license "MIT"
  version "v0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/GustavoDGoat/DOMA/releases/download/v0.1.0/doma-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_ARM64"
    else
      url "https://github.com/GustavoDGoat/DOMA/releases/download/v0.1.0/doma-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_X86_64"
    end
  end

  on_linux do
    url "https://github.com/GustavoDGoat/DOMA/releases/download/v0.1.0/doma-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_LINUX"
  end

  def install
    bin.install "doma"
  end

  test do
    assert_match "DOMA", shell_output("#{bin}/doma --version 2>&1", 1)
  end
end
