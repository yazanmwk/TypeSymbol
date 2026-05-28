class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.16"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.16/typesymbol-v0.1.16-aarch64-apple-darwin.tar.gz"
      sha256 "4bf04c54754d42206aaf9a9d53f1b1ce993dbde4ede4a759d27557836e2687da"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.16/typesymbol-v0.1.16-x86_64-apple-darwin.tar.gz"
      sha256 "390b38496823b836d5c6009e36b89501d1a364d312ccd71646fd5d23164f6b88"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
