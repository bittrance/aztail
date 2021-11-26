const index = await import(process.argv[2]);

await index.default({ log: console });
