FROM chromadb/chroma:0.4.22

EXPOSE 8000

CMD ["uvicorn", "chromadb.app:app", "--reload", "--workers", "1", "--host", "0.0.0.0", "--port", "8000", "--log-level", "info"]
