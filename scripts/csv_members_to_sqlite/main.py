import sqlite3
import csv
import sys
import os
import struct
from typing import List

# Constants for column mapping
CSV_COLUMNS = {
    "discord_id": 0,
    "username": 1,
    "promo": 3,
    "first_name": 4,
    "last_name": 5,
}


class User:
    def __init__(self, discord_id, username, promo, first_name, last_name):
        self.discord_id = discord_id
        self.username = username
        self.promo = promo
        self.first_name = first_name
        self.last_name = last_name

    def __repr__(self):
        return f"User(discord_id={self.discord_id}, username={self.username}, promo={self.promo}, first_name={self.first_name}, last_name={self.last_name})"


def parse_csv(csv_file):
    users = []
    with open(csv_file, newline="", encoding="utf-8") as file:
        reader = csv.reader(file)
        next(reader)  # Skip header if present
        for row in reader:
            user = User(
                discord_id=row[CSV_COLUMNS["discord_id"]],
                username=row[CSV_COLUMNS["username"]],
                promo=int(row[CSV_COLUMNS["promo"]]),
                first_name=row[CSV_COLUMNS["first_name"]],
                last_name=row[CSV_COLUMNS["last_name"]],
            )
            users.append(user)
    return users


def connect_db(sqlite_file):
    conn = sqlite3.connect(sqlite_file)
    return conn


def generate_32_bits_number():
    b = os.urandom(4)
    num = struct.unpack(">I", b)[0]
    num |= 1 << 31
    return num


def insert_users(conn, users: list[User]):
    cursor = conn.cursor()
    for user in users:
        cursor.execute(
            """
            INSERT INTO USERS (id,username, first_name, last_name, discord_id, promotion, permissions)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """,
            (
                generate_32_bits_number(),
                user.username,
                user.first_name,
                user.last_name,
                user.discord_id,
                user.promo,
                1,  # Student role
            ),
        )
        print(f"Inserted user {user.discord_id}/{user.username}")
    conn.commit()


def get_previous_discord_ids(conn) -> List[str]:
    cursor = conn.cursor()
    cursor.execute("SELECT discord_id FROM USERS")
    rows = cursor.fetchall()
    return [row[0] for row in rows]


def main():
    if len(sys.argv) != 3:
        print("Usage: uv run main.py <sqlite_file> <csv_file>")
        sys.exit(1)

    sqlite_file = sys.argv[1]
    csv_file = sys.argv[2]

    users = parse_csv(csv_file)
    print(f"Parsed {len(users)} users:")

    conn = connect_db(sqlite_file)
    print("Connected to SQLite database.")

    old_discord_ids = get_previous_discord_ids(conn)

    users = [
        user
        for user in users
        if user.promo >= 0
        and user.last_name != ""
        and user.first_name != ""
        and user.discord_id not in old_discord_ids
    ]
    print(f"Filtered down to {len(users)} new users:")
    for user in users:
        print(user)

    # You can now insert data into the database
    # Example: cursor = conn.cursor()
    # INSERT QUERY GOES HERE

    insert_users(conn, users)

    conn.close()
    print("Database connection closed.")


if __name__ == "__main__":
    main()
