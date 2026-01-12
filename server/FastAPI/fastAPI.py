# FastAPI/fastAPI.py
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import List, Dict, Optional
import httpx
import os
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("fastapi_logger")

RUST_HOST = os.getenv("LLAMACPP_HOST", "http://rust-api:8080")  # Rust service URL

from filter import is_harmful
from llm_guard.input_scanners import PromptInjection, Toxicity

# Check if scanners are enabled (default: False to speed up init)
ENABLE_SCANNERS = os.getenv("ENABLE_SCANNERS", "false").lower() == "true"

scanners = []
if ENABLE_SCANNERS:
    try:
        logger.info("Initializing LLM-Guard scanners...")
        scanners = [PromptInjection(), Toxicity()]
    except Exception as e:
        logger.error(f"Failed to initialize LLM-Guard scanners: {e}")
else:
    logger.info("Scanners disabled via ENABLE_SCANNERS=false")

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class ChatRequest(BaseModel):
    prompt: str
    refusal_message: str = None
    messages: Optional[List[Dict[str, str]]] = None

DEFAULT_REFUSAL = "The prompt contains content that is not allowed. I cannot assist with topics related to restricted content."

@app.post("/chat")
async def chat(request: ChatRequest):
    logger.info(f"Received prompt: {request.prompt}")
    
    refusal = request.refusal_message or DEFAULT_REFUSAL

    # Layer 1: Keyword/Profanity Filter (Fast)
    if is_harmful(request.prompt):
        logger.warning(f"Blocked by keyword filter: {request.prompt}")
        return {"response": refusal}
    
    # Layer 2: LLM-Guard Scanners (Intent/Injection)
    if ENABLE_SCANNERS:
        for scanner in scanners:
            try:
                _, is_valid, risk_score = scanner.scan(request.prompt)
            except Exception as e:
                logger.error(f"Error during scanning with {scanner.__class__.__name__}: {e}")
                # In case of other errors in scanner, we let it pass to the next layer
                continue
                
            if not is_valid:
                logger.warning(f"Blocked by {scanner.__class__.__name__} (risk: {risk_score}): {request.prompt}")
                return {"response": f"{refusal} ({scanner.__class__.__name__})"}
    try:
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                f"{RUST_HOST}/v1/chat",
                json={
                    "prompt": request.prompt,
                    "messages": request.messages
                }
            )
            logger.info(f"Rust API response status: {resp.status_code}")
            resp.raise_for_status()
            data = resp.json()
            logger.info(f"Rust API response data: {data}")
            return data
    except HTTPException:
        # Re-raise HTTPExceptions (like the 400s from scanners) so they reach the client
        raise
    except httpx.HTTPStatusError as e:
        logger.error(f"HTTPStatusError: {e.response.status_code} - {e.response.text}")
        raise HTTPException(status_code=e.response.status_code, detail=e.response.text)
    except httpx.RequestError as e:
        logger.error(f"RequestError: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Request error: {str(e)}")
    except Exception as e:
        logger.exception("Unexpected error")
        raise HTTPException(status_code=500, detail=str(e))
