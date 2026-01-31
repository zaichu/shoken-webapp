interface ImportMetaEnv {
    readonly VITE_SHOKEN_WEBAPI_API_URL: string;
    readonly DEV: boolean;
    readonly PROD: boolean;
    readonly MODE: string;
}

interface ImportMeta {
    readonly env: ImportMetaEnv;
}