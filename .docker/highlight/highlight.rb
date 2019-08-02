#!/usr/bin/env ruby

require 'rack'
require 'rouge'
require 'faye/websocket'

Faye::WebSocket.load_adapter('thin')

class Highlight
  @html = nil
  @lexer = nil

  def initialize(kind, name)
    if kind == 'file'
      lexers = Rouge::Guesser.guess([Rouge::Guessers::Filename.new(name)], Rouge::Lexer.all)
      @lexer = lexers[0] || Rouge::Lexers::PlainText
    elsif kind == 'snippet'
      @lexer = Rouge::Lexer.find(name) || Rouge::Lexers::PlainText
    else
      raise ArgumentError, 'kind must be "file" or "snippet"'
    end

    @html = Rouge::Formatters::HTML.new
  end

  def highlight(code)
    @html.format(@lexer.lex(code))
  end
end

class Server
  KEEPALIVE_TIME = 15

  def call(env)
    if Faye::WebSocket.websocket?(env)
      ws = Faye::WebSocket.new(env, nil, {ping: KEEPALIVE_TIME})

      ws.on :message do |event|
        id, lang, type, source = event.data.split("\n", 4)
        begin
          hl = Highlight.new(type, lang)
        rescue ArgumentError
          # send nothing
        else
          ws.send("#{id}\n#{hl.highlight(source)}")
        end
      end

      ws.on :close do |event|
        ws = nil
      end

      return ws.rack_response
    else
      ['400', {}, []]
    end
  end
end

Rack::Handler::Thin.run Server.new
