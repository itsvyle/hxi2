import { z } from "zod";
export const TransactionConfigSchema = z.object({
    server_id: z.string(),
    roles: z.record(z.string(), z.string()),
    actions: z.array(
        z.object({
            remove: z.array(z.string()),
            add: z.array(z.string()),

            select: z
                .object({
                    has: z.array(z.string()).optional(),
                    // WARNING: NOT IMPLEMENTED
                    has_not: z.array(z.string()).optional(),
                })
                .refine(
                    (data) => {
                        const hasLength = data.has && data.has.length > 0;
                        const hasNotLength =
                            data.has_not && data.has_not.length > 0;
                        return hasLength || hasNotLength;
                    },
                    {
                        message:
                            "The 'select' object must contain at least one of 'has' or 'has_not' with elements.",
                        path: ["has"],
                    },
                ),
        }),
    ),
});
export type TransactionConfig = z.infer<typeof TransactionConfigSchema>;

export const TransactionRecordSchema = z.object({
    server_id: z.string(),
    roles: z.record(z.string(), z.string()),
    users: z.record(
        z.string(),
        z.object({
            added: z.array(z.string()),
            removed: z.array(z.string()),
            debug_name: z.string().optional(),
        }),
    ),
});

export type TransactionRecord = z.infer<typeof TransactionRecordSchema>;
