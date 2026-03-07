FROM node:18-alpine

WORKDIR /app

COPY package.json ./
RUN npm install --only=production

COPY . .

# Create data directory for CSV storage
RUN mkdir -p data

EXPOSE 80

CMD ["node", "server.js"]
