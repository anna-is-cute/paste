# FROM ruby:alpine # can't use alpine because ffi shared libraries
FROM ruby

# RUN apk add --no-cache build-base
RUN apt-get update && apt-get install --assume-yes --no-install-recommends build-essential

RUN gem install sidekiq ffi

ADD sidekiq.sh /sidekiq.sh

WORKDIR /paste

STOPSIGNAL SIGKILL

CMD /sidekiq.sh
