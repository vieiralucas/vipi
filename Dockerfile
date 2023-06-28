FROM node:latest

RUN npm i -g http-server

COPY . .

EXPOSE 8080

CMD ["http-server"]
