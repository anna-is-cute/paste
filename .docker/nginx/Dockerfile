FROM nginx:mainline-alpine

RUN apk add --update nginx
RUN rm -rf /var/cache/apk/* && rm -rf /tmp/*

COPY nginx.conf /etc/nginx/
COPY app.conf /etc/nginx/conf.d/default.conf

RUN adduser -D -g '' -G www-data www-data

CMD ["nginx"]
