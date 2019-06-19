require 'rubygems'
require 'rouge'

class Highlight
  def highlight_snippet(lexer_name, source)
    lexer = Rouge::Lexer.find(lexer_name) || Rouge::Lexers::PlainText

    highlight(lexer, source)
  end

  def highlight_file(file_name, source)
    lexers = Rouge::Guesser.guess([Rouge::Guessers::Filename.new(file_name)], Rouge::Lexer.all)
    lexer = lexers[0] || Rouge::Lexers::PlainText

    highlight(lexer, source)
  end

  def highlight(lexer, source)
    html = Rouge::Formatters::HTML.new

    source.split("\n").map do |line|
      html.format(lexer.lex(line))
    end
  end
end
