import { zodToConvex } from "convex-helpers/server/zod";
import { defineSchema, defineTable } from "convex/server";
import { z } from "zod";

export const Score = z.object({
  name: z.string().max(2),
  score: z.number(),
  map: z.union([
    z.literal("dual vision"),
    z.literal("clockback"),
    z.literal("curl valley"),
  ]),
});

export default defineSchema({
  scores: defineTable(zodToConvex(Score)).index("by_map_score", [
    "map",
    "score",
  ]),
});
