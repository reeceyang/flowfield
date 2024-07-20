import { Infer } from "convex/values";
import { Doc } from "./_generated/dataModel";
import {
  httpAction,
  internalMutation,
  internalQuery,
} from "./_generated/server";
import { internal } from "./_generated/api";
import schema, { Score } from "./schema";

export const _newScore = internalMutation({
  args: schema.tables.scores.validator,
  handler: async (ctx, args) => {
    await ctx.db.insert("scores", args);
  },
});

export const _getTopScores = internalQuery({
  args: {},
  handler: async (ctx) => {
    const topScores: Partial<
      Record<
        Infer<typeof schema.tables.scores.validator.fields.map>,
        Doc<"scores">[]
      >
    > = {};
    for (const { value: map } of schema.tables.scores.validator.fields.map
      .members) {
      topScores[map] = await ctx.db
        .query("scores")
        .withIndex("by_map_score")
        .filter((q) => q.eq(q.field("map"), map))
        .order("desc")
        .take(5);
    }
    return topScores;
  },
});

export const newScore = httpAction(async (ctx, request) => {
  const score = Score.parse(await request.json());

  await ctx.runMutation(internal.scores._newScore, score);

  return new Response(null, {
    status: 200,
  });
});

export const getTopScores = httpAction(async (ctx, request) => {
  const topScores = await ctx.runQuery(internal.scores._getTopScores);

  return new Response(JSON.stringify(topScores), {
    status: 200,
  });
});
