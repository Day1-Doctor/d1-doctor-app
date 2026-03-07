# typed: false
# frozen_string_literal: true

# Homebrew formula for Day1 Doctor CLI
#
# Install:  brew install day1doctor/tap/d1
# Upgrade:  brew upgrade d1
#
# This formula downloads the pre-built universal macOS binary from GitHub Releases.
# For Linux, use the curl installer: scripts/install.sh

class D1 < Formula
  desc "Day1 Doctor CLI - AI-powered system setup assistant"
  homepage "https://github.com/Day1-Doctor/d1-doctor-app"
  version "0.1.0"
  license "MIT"

  on_macos do
    url "https://github.com/Day1-Doctor/d1-doctor-app/releases/download/v#{version}/d1-macos-universal.tar.gz"
    # sha256 will be filled by CI or maintainer after first release
    # sha256 "PLACEHOLDER"
  end

  on_linux do
    on_intel do
      url "https://github.com/Day1-Doctor/d1-doctor-app/releases/download/v#{version}/d1-linux-x86_64.tar.gz"
      # sha256 "PLACEHOLDER"
    end

    on_arm do
      url "https://github.com/Day1-Doctor/d1-doctor-app/releases/download/v#{version}/d1-linux-arm64.tar.gz"
      # sha256 "PLACEHOLDER"
    end
  end

  def install
    bin.install "d1-doctor-cli" => "d1"
  end

  test do
    assert_match "d1-doctor-cli", shell_output("#{bin}/d1 --version")
  end
end
