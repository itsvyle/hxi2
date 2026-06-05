// print out all command line arguments
import process from "node:process";
import { Client, GatewayIntentBits } from "discord.js";
import { TransactionConfigSchema, type TransactionRecord } from "./schemas";

if (process.argv.length < 4) {
    console.error(
        "Usage: node index.js <name> <mode = generate-transaction|apply-transaction|revert-transaction> <transaction file or rules file>",
    );
    process.exit(1);
}

const mode = process.argv[2];
if (!mode) {
    console.error("Mode is required");
    process.exit(1);
}
const filePath = process.argv[3];
if (!filePath) {
    console.error("File is required");
    process.exit(1);
}

const possibleModes = [
    "generate-transaction",
    "apply-transaction",
    "revert-transaction",
];
if (!possibleModes.includes(mode)) {
    console.error(
        `Invalid mode: ${mode}. Possible modes are: ${possibleModes.join(", ")}`,
    );
    process.exit(1);
}

const discordToken = process.env.DISCORD_TOKEN;
if (!discordToken) {
    console.error("DISCORD_TOKEN environment variable is not set");
    process.exit(1);
}

const client = new Client({
    intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildMembers],
});

client.once("clientReady", async () => {
    console.log("Logged in to Discord");
    let fileContents: string;
    try {
        const file = Bun.file(filePath);

        fileContents = await file.text();
    } catch (error) {
        console.error(`Error reading file: ${error}`);
        process.exit(1);
    }

    switch (mode) {
        case "generate-transaction":
            await modeGenerateTransaction(fileContents);
            break;
        case "apply-transaction":
            await modeApplyTransaction(fileContents);
            break;
        case "revert-transaction":
            await modeRevertTransaction(fileContents);
            break;
    }

    process.exit(0);
});

async function modeGenerateTransaction(fileContents: string) {
    const configParsing = TransactionConfigSchema.safeParse(
        JSON.parse(fileContents),
    );
    if (!configParsing.success) {
        console.error(`Invalid config file: ${configParsing.error}`);
        process.exit(1);
    }
    const config = configParsing.data;

    const guild = client.guilds.cache.get(config.server_id);
    if (!guild) {
        console.error(`Server not found: ${config.server_id}`);
        process.exit(1);
    }

    for (let role_pretty_name in config.roles) {
        const role_id = config.roles[role_pretty_name];
        if (!role_id) continue;
        const role = guild.roles.cache.get(role_id);
        if (!role) {
            console.error(
                `Role not found on the server: ${role_pretty_name} (given id: ${role_id})`,
            );
            process.exit(1);
        }
    }

    let get_role_id = (pretty_name: string) => {
        const role_id = config.roles[pretty_name];
        if (!role_id) {
            console.error(`Role not found: ${pretty_name}`);
            process.exit(1);
        }
        return role_id;
    };

    let members = new Map<string, { roles: string[]; username: string }>();
    await client.guilds.cache
        .get(config.server_id)
        ?.members.fetch()
        .then((fetchedMembers) => {
            fetchedMembers.forEach((member) => {
                members.set(member.id, {
                    roles: member.roles.cache.map((role) => role.id),
                    username: member.user.username,
                });
            });
        });

    let transaction: TransactionRecord = {
        roles: config.roles,
        server_id: config.server_id,
        users: {},
    };

    for (let action of config.actions) {
        let selectedMembers = new Set<string>();
        if (action.select.has) {
            for (let role_pretty_name of action.select.has) {
                const role_id = get_role_id(role_pretty_name);
                members.forEach((member, member_id) => {
                    if (member.roles.includes(role_id)) {
                        selectedMembers.add(member_id);
                    }
                });
            }
        }
        if (action.select.has_not) {
            console.error("has_not is not implemented yet");
            process.exit(1);
        }

        selectedMembers.forEach((member_id) => {
            if (!transaction.users[member_id]) {
                transaction.users[member_id] = {
                    added: [],
                    removed: [],
                    debug_name: members.get(member_id)?.username || "Unknown",
                };
            }
            transaction.users[member_id].added.push(...action.add);
            transaction.users[member_id].removed.push(...action.remove);
        });
    }

    let addingRolesToUsersCount = 0;
    let removingRolesFromUsersCount = 0;
    for (let user_id in transaction.users) {
        if (transaction.users[user_id]!.added.length > 0) {
            addingRolesToUsersCount += 1;
        }
        if (transaction.users[user_id]!.removed.length > 0) {
            removingRolesFromUsersCount += 1;
        }
    }

    console.log("Generated transaction:");
    console.log(`Adding roles to ${addingRolesToUsersCount} users`);
    console.log(`Removing roles from ${removingRolesFromUsersCount} users`);

    const outputFilePath = `transaction-${new Date().toISOString().replace(/[:.]/g, "-")}.json`;
    await Bun.write(outputFilePath, JSON.stringify(transaction, null, 2));

    console.log(`Transaction written to ${outputFilePath}`);
    console.log("Apply it with:");
    console.log(`bun run index.ts apply-transaction ${outputFilePath}`);
}
async function executeTransaction(fileContents: string, isRevert: boolean) {
    const transaction = JSON.parse(fileContents) as TransactionRecord;

    const guild = client.guilds.cache.get(transaction.server_id);
    if (!guild) {
        console.error(`Server not found: ${transaction.server_id}`);
        process.exit(1);
    }

    const modeLabel = isRevert ? "Reverting" : "Applying";
    console.log(
        `${modeLabel} transaction on server: ${guild.name} (${guild.id})`,
    );

    for (const [userId, userRecord] of Object.entries(transaction.users)) {
        try {
            const member = await guild.members.fetch(userId);

            const rolesToGive = isRevert
                ? userRecord.removed
                : userRecord.added;
            const rolesToTake = isRevert
                ? userRecord.added
                : userRecord.removed;

            const roleIdsToAdd = rolesToGive
                .map((roleName) => transaction.roles[roleName])
                .filter((id): id is string => !!id);

            const roleIdsToRemove = rolesToTake
                .map((roleName) => transaction.roles[roleName])
                .filter((id): id is string => !!id);

            if (roleIdsToAdd.length > 0) {
                await member.roles.add(roleIdsToAdd);
                console.log(
                    `[${isRevert ? "REVERT-" : ""}ADD] Given [${rolesToGive.join(", ")}] to ${userRecord.debug_name}`,
                );
            }

            if (roleIdsToRemove.length > 0) {
                await member.roles.remove(roleIdsToRemove);
                console.log(
                    `[${isRevert ? "REVERT-" : ""}REMOVE] Taken [${rolesToTake.join(", ")}] from ${userRecord.debug_name}`,
                );
            }
        } catch (error) {
            console.error(
                `Error processing updates for user ${userRecord.debug_name} (${userId}):`,
                error,
            );
        }
    }

    console.log(
        `Transaction ${isRevert ? "reversion" : "application"} complete.`,
    );
}

async function modeApplyTransaction(fileContents: string) {
    await executeTransaction(fileContents, false);
}

async function modeRevertTransaction(fileContents: string) {
    await executeTransaction(fileContents, true);
}

client.login(discordToken);
