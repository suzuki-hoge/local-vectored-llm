FROM ollama/ollama:0.6.8

# 中国語の軽量モデル ( 最低限の動作確認用のため除外 )
# RUN /bin/ollama serve & sleep 5 && ollama run deepseek-r1:1.5b

# サイバーエージェントの軽量モデル ( 認証が必要で除外 )
# RUN /bin/ollama serve & sleep 5 && ollama run hf.co/mmnga/cyberagent-DeepSeek-R1-Distill-Qwen-3B-Japanese-gguf


# サイバーエージェントの中型モデル ( 認証が必要で除外 )
# RUN /bin/ollama serve & sleep 5 && ollama run hf.co/mmnga/cyberagent-DeepSeek-R1-Distill-Qwen-7B-Japanese-gguf

# サイバーエージェントの大型モデル ( 標準設定だとメモリが足りず除外 )
RUN /bin/ollama serve & sleep 5 && ollama run hf.co/mmnga/cyberagent-DeepSeek-R1-Distill-Qwen-14B-Japanese-gguf
