# Homebrew Formula for apo
#
# Source of truth in-repo. Checksums are filled by
# scripts/update-packaging-checksums.sh from the GitHub Release SHA256SUMS.
# Copy to baoulo/homebrew-tap as Formula/apo.rb after each release.
class Apo < Formula
  desc "APO — Engineering Evidence Platform. Repository hygiene, knowledge, and AI evidence."
  homepage "https://github.com/baoulo/apo"
  version "0.3.2"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-macos-aarch64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-macos-x86_64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-linux-aarch64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
    on_intel do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-linux-x86_64.tar.gz"
      # sha256 "REPLACE_AFTER_RELEASE"
    end
  end

  def install
    bin.install "apo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/apo --version")
  end
end
