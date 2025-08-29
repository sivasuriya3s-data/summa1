# Use the official Node.js runtime as the base image
FROM node:18-alpine

# Set the working directory inside the container
WORKDIR /app

# Copy package.json and package-lock.json (if available)
COPY package*.json ./

# Install all dependencies (including dev dependencies for Vite)
RUN npm ci

# Copy the rest of the application code
COPY . .

# Create a non-root user for security
RUN addgroup -g 1001 -S nodejs && \
    adduser -S nextjs -u 1001

# Change ownership of the app directory to the nodejs user
RUN chown -R nextjs:nodejs /app
USER nextjs

# Expose the port the app runs on (Vite default is 5173)
EXPOSE 5173

# Set environment to development (needed for Vite dev server)
ENV NODE_ENV=development

# Command to run the application with host binding for Docker
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
