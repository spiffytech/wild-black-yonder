import { Level } from "level";

interface Record {
  subject: string;
  predicate: string;
  object: string;
}

export const db = new Level<string, any>("my-db", { valueEncoding: "json" });

export const spo = ({ subject, predicate, object }: Record) =>
  `spo::${subject}::${predicate}::${object}`;
export const sop = ({ subject, predicate, object }: Record) =>
  `sop::${subject}::${object}::${predicate}`;
export const ops = ({ subject, predicate, object }: Record) =>
  `ops::${object}::${predicate}::${subject}`;
export const osp = ({ subject, predicate, object }: Record) =>
  `osp::${object}::${subject}::${predicate}`;
export const pso = ({ subject, predicate, object }: Record) =>
  `pso::${predicate}::${subject}::${object}`;
export const pos = ({ subject, predicate, object }: Record) =>
  `pos::${predicate}::${object}::${subject}`;

const range = (
  arrangement: "spo" | "sop" | "ops" | "osp" | "pso" | "pos",
  [one, two]: [string, string]
) => {
  const prefix = `${arrangement}::${one}::${two}::`;
  return { gt: prefix, lt: `${prefix}\xff` };
};

export const sp = ({
  subject,
  predicate,
}: Pick<Record, "subject" | "predicate">) => {
  return range("spo", [subject, predicate]);
};
export const so = ({ subject, object }: Pick<Record, "subject" | "object">) => {
  return range("sop", [subject, object]);
};
export const op = ({
  object,
  predicate,
}: Pick<Record, "object" | "predicate">) => {
  return range("ops", [object, predicate]);
};
export const os = ({ object, subject }: Pick<Record, "object" | "subject">) => {
  return range("osp", [object, subject]);
};
export const ps = ({
  predicate,
  subject,
}: Pick<Record, "predicate" | "subject">) => {
  return range("pso", [predicate, subject]);
};
export const po = ({
  predicate,
  object,
}: Pick<Record, "predicate" | "object">) => {
  return range("pos", [predicate, object]);
};

export const manyKeys = (record: Record) =>
  [spo, sop, ops, osp, pso, pos].map((arrangement) => arrangement(record));

export const batch = (type: "put" | "del", record: Record, value: unknown) =>
  manyKeys(record).map((key) => ({ type, key, value }));
