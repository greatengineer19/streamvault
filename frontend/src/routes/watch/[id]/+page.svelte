<script lang="ts">
    import { onMount } from "svelte";
    import { getStreamUrl, type StreamUrlResponse } from "$lib/api";
    import type { PageData } from "./$types";

    export let data: PageData;

    let streamData: StreamUrlResponse | null = null;
    let error = "";
    let loading = true;
    let copied = false;

    onMount(async () => {
        try {
            streamData = await getStreamUrl(data.id);
        } catch (e: any) {
            error = e.message ?? "Video not found or not ready yet.";
        } finally {
            loading = false;
        }
    });

    async function copyLink() {
        await navigator.clipboard.writeText(window.location.href);
        copied = true;
        setTimeout(() => (copied = false), 2000);
    }
</script>

<svelte:head>
    <title>{streamData?.filename ?? "StreamVault"}</title>
</svelte:head>

<div class="page">
    <nav>
        <a href="/" class="back-link">
            <span class="logo-mark">▶</span>
            <span class="logo-text"
                >STREAM<span class="accent">VAULT</span></span
            >
        </a>
        {#if streamData}
            <button class="share-btn" on:click={copyLink}>
                {copied ? "✓ COPIED" : "⎘ SHARE"}
            </button>
        {/if}
    </nav>

    <main>
        {#if loading}
            <div class="state-box">
                <div class="loading-ring"></div>
                <p class="state-text">Loading video...</p>
            </div>
        {:else if error}
            <div class="state-box error">
                <div class="error-code">404</div>
                <p class="error-msg">{error}</p>
                <a href="/" class="home-link">← Upload a video</a>
            </div>
        {:else if streamData}
            <div class="player-wrap">
                <!-- Native HTML5 video — browser handles range requests automatically -->
                <video
                    class="player"
                    src={streamData.stream_url}
                    controls
                    preload="metadata"
                    playsinline
                >
                    <track kind="captions" />
                    Your browser does not support HTML5 video.
                </video>
            </div>

            <div class="meta">
                <div class="meta-left">
                    <h1 class="filename">{streamData.filename}</h1>
                    {#if streamData.mime_type}
                        <span class="badge"
                            >{streamData.mime_type
                                .split("/")[1]
                                .toUpperCase()}</span
                        >
                    {/if}
                </div>
                <div class="meta-right">
                    <a href="/" class="upload-link">Upload another →</a>
                </div>
            </div>
        {/if}
    </main>
</div>

<style>
    .page {
        min-height: 100vh;
        width: 100%;
        max-width: 100%;
        display: flex;
        flex-direction: column;
        background: var(--bg);
        overflow-x: hidden;
    }

    /* Nav */
    nav {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem 1.25rem;
        border-bottom: 1px solid var(--border);
    }

    .back-link {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        text-decoration: none;
        transition: opacity 0.15s;
    }

    .back-link:hover {
        opacity: 0.7;
    }

    .logo-mark {
        font-size: 1rem;
        color: var(--accent);
    }

    .logo-text {
        font-family: var(--sans);
        font-size: 1.1rem;
        font-weight: 800;
        letter-spacing: 0.15em;
    }

    .accent {
        color: var(--accent);
    }

    .share-btn {
        font-family: var(--mono);
        font-size: 0.7rem;
        letter-spacing: 0.1em;
        padding: 0.5rem 1rem;
        background: transparent;
        border: 1px solid var(--border);
        color: var(--text-muted);
        transition:
            border-color 0.15s,
            color 0.15s;
        min-height: 44px;
    }

    .share-btn:hover {
        border-color: var(--accent);
        color: var(--accent);
    }

    /* Main */
    main {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 1.5rem 1rem;
        gap: 1.25rem;
    }

    /* States */
    .state-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        padding: 2rem 1rem;
    }

    .loading-ring {
        width: 40px;
        height: 40px;
        border: 2px solid var(--border);
        border-top-color: var(--accent);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .state-text {
        font-family: var(--mono);
        font-size: 0.8rem;
        color: var(--text-muted);
        letter-spacing: 0.05em;
    }

    .error-code {
        font-family: var(--sans);
        font-size: 4rem;
        font-weight: 800;
        color: var(--border);
        line-height: 1;
    }

    .error-msg {
        font-family: var(--mono);
        font-size: 0.85rem;
        color: var(--text-muted);
        text-align: center;
    }

    .home-link {
        font-family: var(--mono);
        font-size: 0.8rem;
        color: var(--accent);
        margin-top: 0.5rem;
        min-height: 44px;
        display: flex;
        align-items: center;
    }

    /* Player */
    .player-wrap {
        width: 100%;
        max-width: 960px;
        background: #000;
        position: relative;
        line-height: 0;
    }

    .player {
        width: 100%;
        /* On mobile use more screen height for the video */
        max-height: 60vh;
        display: block;
        background: #000;
        outline: none;
    }

    /* Meta bar */
    .meta {
        width: 100%;
        max-width: 960px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .meta-left {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        flex-wrap: wrap;
    }

    .filename {
        font-family: var(--sans);
        font-size: 1rem;
        font-weight: 700;
        color: var(--text);
        word-break: break-all;
    }

    .badge {
        font-family: var(--mono);
        font-size: 0.6rem;
        letter-spacing: 0.1em;
        padding: 0.2rem 0.5rem;
        border: 1px solid var(--border);
        color: var(--text-muted);
        flex-shrink: 0;
    }

    .upload-link {
        font-family: var(--mono);
        font-size: 0.75rem;
        color: var(--text-muted);
        transition: color 0.15s;
        white-space: nowrap;
        min-height: 44px;
        display: flex;
        align-items: center;
    }

    .upload-link:hover {
        color: var(--accent);
    }

    /* ── Mobile breakpoint ───────────────────────────────────────────────────── */
    @media (max-width: 480px) {
        nav {
            padding: 0.875rem 1rem;
        }

        .logo-text {
            font-size: 0.95rem;
            letter-spacing: 0.1em;
        }

        main {
            padding: 1rem 0.875rem;
            justify-content: flex-start;
        }

        .player {
            /* On mobile, use full width and natural aspect ratio */
            max-height: none;
        }

        .meta {
            flex-direction: column;
            align-items: flex-start;
            gap: 0.5rem;
        }

        .filename {
            font-size: 0.9rem;
        }
    }
</style>
