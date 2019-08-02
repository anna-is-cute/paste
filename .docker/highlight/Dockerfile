FROM ruby:alpine

RUN apk add --no-cache openssl-dev build-base

RUN gem install rouge rack thin faye-websocket

RUN apk del openssl-dev build-base

ADD highlight.rb /highlight.rb

ENV RACK_ENV production

CMD ["ruby", "/highlight.rb"]
