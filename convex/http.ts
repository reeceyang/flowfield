import { httpRouter } from "convex/server";
import { getTopScores, newScore } from "./scores";

const http = httpRouter();

http.route({
  path: "/topScores",
  method: "GET",
  handler: getTopScores,
});

http.route({
  path: "/newScore",
  method: "POST",
  handler: newScore,
});

// Convex expects the router to be the default export of `convex/http.js`.
export default http;
