DROP TABLE IF EXISTS "shorts";
CREATE TABLE "public"."shorts" (
    "id" text NOT NULL,
    "long_url" text NOT NULL
);

CREATE UNIQUE INDEX shorts_id ON public.shorts USING btree (id);