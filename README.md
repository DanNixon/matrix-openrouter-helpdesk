# matrix-openrouter-helpdesk

A Matrix bot written in Rust that responds to messages with AI-generated answers using OpenRouter.

## Features

- **Automatic Room Invites**: The bot automatically accepts room invitations
- **Message Processing**: Responds to messages in the format `@bot_username question` (case insensitive)
- **Visual Feedback**: Adds a "👀" (eyes) emoji reaction to messages it's processing
- **AI-Powered Responses**: Uses OpenRouter API to generate intelligent responses
- **Self-Aware**: Ignores its own messages to prevent loops
- **Prometheus Metrics**: Exposes metrics in Prometheus format for monitoring

## Prerequisites

- Rust 1.70 or later
- A Matrix account for the bot
- An OpenRouter API key

## Configuration

The bot is configured using environment variables:

- `MATRIX_HOMESERVER_URL`: The Matrix homeserver URL (e.g., `https://matrix.org`)
- `MATRIX_USERNAME`: The bot's Matrix username (e.g., `@helpdesk:matrix.org`)
- `MATRIX_PASSWORD`: The bot's Matrix password
- `OPENROUTER_API_KEY`: Your OpenRouter API key
- `OPENROUTER_MODEL` (optional): The OpenRouter model to use (defaults to `openai/gpt-3.5-turbo`)
- `METRICS_PORT` (optional): Port for Prometheus metrics endpoint (defaults to `9090`)

## Building

```bash
cargo build --release
```

## Running

```bash
export MATRIX_HOMESERVER_URL="https://matrix.org"
export MATRIX_USERNAME="@your-bot:matrix.org"
export MATRIX_PASSWORD="your-password"
export OPENROUTER_API_KEY="your-openrouter-api-key"
export OPENROUTER_MODEL="openai/gpt-3.5-turbo"  # optional

cargo run --release
```

Or using environment file:

```bash
# Create a .env file with your configuration
cat > .env << EOF
MATRIX_HOMESERVER_URL=https://matrix.org
MATRIX_USERNAME=@your-bot:matrix.org
MATRIX_PASSWORD=your-password
OPENROUTER_API_KEY=your-openrouter-api-key
OPENROUTER_MODEL=openai/gpt-3.5-turbo
EOF

# Load environment variables and run
set -a; source .env; set +a
cargo run --release
```

## Usage

1. Invite the bot to a Matrix room
2. The bot will automatically join the room
3. Send a message mentioning the bot: `@bot_username What is the weather today?`
4. The bot will:
   - React with 👀 to indicate it's processing
   - Send a quoted reply with the AI-generated response

## Example

```
You: @helpdesk:matrix.org What is Rust?
Bot: 👀 (reaction)
Bot: > What is Rust?
     Rust is a systems programming language focused on safety, speed, and concurrency...
```

## Metrics

The bot exposes Prometheus metrics on the configured port (default: 9090). The metrics endpoint is available at `http://localhost:9090/metrics`.

### Available Metrics

- `helpdesk_requests_total`: Counter tracking total number of requests processed
  - Labels:
    - `matrix_user`: The Matrix user ID who sent the request
    - `matrix_room`: The Matrix room ID where the request was made
    - `result`: Either `success` or `failure`

Example Prometheus query:
```promql
# Total successful requests
helpdesk_requests_total{result="success"}

# Request rate per room
rate(helpdesk_requests_total[5m])
```

## License

This project is open source and available under your preferred license.