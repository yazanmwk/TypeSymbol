class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.18"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.18/typesymbol-v0.1.18-aarch64-apple-darwin.tar.gz"
      sha256 "9eb6a9a4ff57cb6374042d6c6a4d22718e7b0dda567ab99e7e6969eaa451de66"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.18/typesymbol-v0.1.18-x86_64-apple-darwin.tar.gz"
      sha256 "b21897fd2391b84325196c5e752b565b8e5b260fd160f4bc26f365743cb47726"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
