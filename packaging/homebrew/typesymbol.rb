class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.4"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.4/typesymbol-v0.1.4-aarch64-apple-darwin.tar.gz"
      sha256 "e7180d45473561ab4850b2cdd82550de50fc82e5ed2743b50e1ee7a9a6c46c03"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.4/typesymbol-v0.1.4-x86_64-apple-darwin.tar.gz"
      sha256 "e7180d45473561ab4850b2cdd82550de50fc82e5ed2743b50e1ee7a9a6c46c03"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
