class Typesymbol < Formula
  desc "System-wide math shorthand daemon"
  homepage "https://github.com/yazanmwk/TypeSymbol"
  version "0.1.9"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.9/typesymbol-v0.1.9-aarch64-apple-darwin.tar.gz"
      sha256 "56ab1af206394ce7def2c7bd7cf7dfcd96ef406cc7fdbd4ea3162fd6805753ee"
    else
      url "https://github.com/yazanmwk/TypeSymbol/releases/download/v0.1.9/typesymbol-v0.1.9-x86_64-apple-darwin.tar.gz"
      sha256 "14f547e31149c9abd35e840813cef5aaa45b29c47ba726a7602ea797cb9c1800"
    end
  end

  def install
    bin.install "typesymbol"
  end

  test do
    assert_match "α → β", shell_output("#{bin}/typesymbol test \"alpha -> beta\"")
  end
end
