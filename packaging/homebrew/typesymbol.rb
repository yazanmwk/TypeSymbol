class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.19"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.19/typesymbol-v0.1.19-aarch64-apple-darwin.tar.gz"
      sha256 "3d0c2441e785472cb4abf203b6280188ceb689915e37c377ce6890328fae2f9b"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.19/typesymbol-v0.1.19-x86_64-apple-darwin.tar.gz"
      sha256 "bcdc6fde6ec3f4a0bdf08dffc9e1fcf8cb9d0ce375f84c9c357b40b7bbec1aa0"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
