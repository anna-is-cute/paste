FROM ruby:alpine

RUN gem install rouge rack

ADD highlight.rb /highlight.rb

ENV RACK_ENV production

CMD ["ruby", "/highlight.rb"]
