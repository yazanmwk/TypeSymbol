class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.19"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.19/typesymbol-v0.1.19-aarch64-apple-darwin.tar.gz"
      sha256 "37b83dca6b0df43164501f5f180d86328feff31dd549a9b9bb5a0a66ba9ffaf3"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.19/typesymbol-v0.1.19-x86_64-apple-darwin.tar.gz"
      sha256 "46a39f40c40fc67b5a178a5ac46aaea46989da8f67ec023cb3d8bf287a064b92"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
