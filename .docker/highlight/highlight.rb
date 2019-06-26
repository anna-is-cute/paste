#!/usr/bin/env ruby

require 'rack'
require 'rouge'

class Highlight
  @html = nil
  @lexer = nil

  def initialize(kind, name)
    if kind == 'file' then
      lexers = Rouge::Guesser.guess([Rouge::Guessers::Filename.new(name)], Rouge::Lexer.all)
      @lexer = lexers[0] || Rouge::Lexers::PlainText
    elsif kind == 'snippet' then
      @lexer = Rouge::Lexer.find(name) || Rouge::Lexers::PlainText
    end

    @html = Rouge::Formatters::HTML.new
  end

  def highlight(code)
    @html.format(@lexer.lex(code))
  end
end

class Server
  def call(env)
    req = Rack::Request.new(env)

    if !req.post?
      return ['400', {}, []]
    end

    if req.path == '/highlight/snippet'
      kind = 'snippet'
    elsif req.path == '/highlight/file'
      kind = 'file'
    else
      return ['404', {}, []]
    end

    name = req.params['name']
    code = req.body.read

    hl = Highlight.new(kind, name)

    ['200', {'Content-Type' => 'text/plain'}, [hl.highlight(code)]]
  end
end

Rack::Handler::WEBrick.run Server.new
