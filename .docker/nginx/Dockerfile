FROM nginx:1.17.5-alpine

RUN apk add --no-cache openssl

RUN openssl dhparam -dsaparam -out /etc/ssl/dhparam.pem 4096
