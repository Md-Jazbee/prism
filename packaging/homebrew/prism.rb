# Homebrew formula draft (P11 Stage A)
# Not yet published to a public tap — fill `url`/`sha256` from a real GitHub Release.
#
# Usage (local tap sketch):
#   brew install --formula ./packaging/homebrew/prism.rb
#
# After first release, prefer a tap formula that pins version + sha256 per bottle/triple.

class Prism < Formula
  desc "Repository intelligence CLI + MCP (compile_context Evidence Packs)"
  homepage "https://github.com/example/prism"
  version "0.0.1"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/example/prism/releases/download/v0.0.1/prism-0.0.1-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
    on_intel do
      url "https://github.com/example/prism/releases/download/v0.0.1/prism-0.0.1-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/example/prism/releases/download/v0.0.1/prism-0.0.1-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
    on_intel do
      url "https://github.com/example/prism/releases/download/v0.0.1/prism-0.0.1-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_RELEASE_SHA256"
    end
  end

  def install
    bin.install "prism"
  end

  test do
    assert_match "Prism", shell_output("#{bin}/prism --help")
  end
end
