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
      sha256 "ea68d9845154788fa3c4b4d175c323d41c7e172f8b08ffe9fd12f2114f2053ec"
    end
    on_intel do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-macos-x86_64.tar.gz"
      sha256 "73232a664bd07e8e421e9cab500f2566456a466e91c6c0c2b9346b3bce481e07"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-linux-aarch64.tar.gz"
      sha256 "ee700af1546b46cb84fd290ff5cd46280e4211b245c1d8238edcb88c7c34be53"
    end
    on_intel do
      url "https://github.com/baoulo/apo/releases/download/v#{version}/apo-v#{version}-linux-x86_64.tar.gz"
      sha256 "4a430fabf82724f2a53fe98b4ddd1b4dea939f2b21e7a4fa06c0f3db8ec0f2ac"
    end
  end

  def install
    bin.install "apo"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/apo --version")
  end
end
