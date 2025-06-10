import discord
import csv
import os
from dotenv import load_dotenv
import sys

# Load bot token from environment file
load_dotenv("scrap_discord_users.env")
DISCORD_BOT_TOKEN = os.getenv("DISCORD_BOT_TOKEN")

# Define the server (guild) ID as a constant
if len(sys.argv) < 2:
    print("Usage: python scrap_discord_users.py <GUILD_ID>")
    sys.exit(1)

GUILD_ID: int = int(sys.argv[1])

PROMO_ROLES = {
    876745282451292180: 2021,
    1273987459121680575: 2022,
    1273987681067597824: 2023,
    1273987780040458240: 2024,
}


class DiscordScraper(discord.Client):
    def __init__(self, intents: discord.Intents):
        super().__init__(intents=intents)

    async def on_ready(self):
        print(f"Logged in as {self.user}")
        guild = self.get_guild(GUILD_ID)
        if guild is None:
            print("Guild not found! Check the GUILD_ID.")
            await self.close()
            return

        print(f"Fetching members from {guild.name}...")
        members = guild.members

        # Write members to CSV
        with open(
            "discord_users.csv", "w", newline="", encoding="utf-8"
        ) as file:
            writer = csv.writer(file)
            writer.writerow(
                ["User ID", "Username", "Server Nickname", "Promo year"]
            )  # CSV headers
            count = 0
            for member in members:
                if member.bot:
                    continue
                promo: int = 0
                for role in member.roles:
                    if role.id in PROMO_ROLES:
                        promo = PROMO_ROLES[role.id]
                        break
                writer.writerow(
                    [member.id, member.name, member.display_name, promo]
                )
                count += 1

        print(f"Successfully saved {count} users to discord_users.csv")
        await self.close()


# Intents required for fetching member lists
intents = discord.Intents.default()
intents.members = True  # This must be enabled in the bot settings

client = DiscordScraper(intents=intents)

if __name__ == "__main__":
    client.run(DISCORD_BOT_TOKEN)
