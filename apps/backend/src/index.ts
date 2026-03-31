import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { betterAuth } from "better-auth";
import { authMiddleware } from "better-auth/hono";
import { S3Client, PutObjectCommand, GetObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
import { v4 as uuidv4 } from "uuid";

const auth = betterAuth({
    database: {
        db: process.env.DATABASE_URL!,
        type: "postgres"
    },
    secret: process.env.AUTH_SECRET!
});

const s3Client = new S3Client({
    region: "auto",
    endpoint: process.env.R2_ENDPOINT!,
    credentials: {
        accessKeyId: process.env.R2_ACCESS_KEY_ID!,
        secretAccessKey: process.env.R2_SECRET_ACCESS_KEY!,
    },
});

const app = new Hono();

app.use("*", authMiddleware(auth));

app.get("/", (c) => {
  return c.text("Hello Hono!");
});

app.post("/upload/presigned", async (c) => {
    const session = c.get("session");
    if (!session) return c.json({ error: "Unauthorized" }, 401);

    const { contentType, fileName } = await c.req.json();
    const key = `${session.user.id}/${uuidv4()}-${fileName}`;

    const command = new PutObjectCommand({
        Bucket: process.env.R2_BUCKET_NAME!,
        Key: key,
        ContentType: contentType,
    });

    const url = await getSignedUrl(s3Client, command, { expiresIn: 3600 });

    return c.json({ url, key });
});

app.post("/process", async (c) => {
    const session = c.get("session");
    if (!session) return c.json({ error: "Unauthorized" }, 401);

    const { key, fileType } = await c.req.json();

    // 1. Generate a GET signed URL for the Python worker to download the file
    const command = new GetObjectCommand({
        Bucket: process.env.R2_BUCKET_NAME!,
        Key: key,
    });
    const downloadUrl = await getSignedUrl(s3Client, command, { expiresIn: 600 });

    // 2. Call Python worker
    const workerUrl = process.env.OCR_WORKER_URL || "http://localhost:8001/process";
    const response = await fetch(workerUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ url: downloadUrl, file_type: fileType }),
    });

    if (!response.ok) {
        return c.json({ error: "Worker processing failed" }, 500);
    }

    const result = await response.json();

    // 3. TODO: Call Rust SmartMerge logic in Phase 4
    
    return c.json(result);
});

serve(
  {
    fetch: app.fetch,
    port: 8000,
  },
  (info) => {
    console.log(`Server is running on http://localhost:${info.port}`);
  }
);
