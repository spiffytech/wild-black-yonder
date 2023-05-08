const bearer = import.meta.env.VITE_BEARER;
if (!bearer) throw new Error("Must supply VITE_BEARER arg");

import * as st from "spacetraders-sdk";
const config = new st.Configuration({
  accessToken: bearer,
});
export default config;
