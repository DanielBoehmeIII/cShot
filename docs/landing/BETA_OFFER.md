# Week 28 — Paid Beta Offer

## Pricing Tiers

### Tier 1: Custom Kit — $29
- One custom-generated one-shot kit
- Choose your genre/vibe, song, or sample
- 40-80 curated, quality-gated sounds
- DAW-ready export (Ableton or FL Studio folders)
- Delivered within 24 hours

### Tier 2: Beta Access — $99
- 5 custom kits
- Early access to new features
- Direct feedback channel with the developer
- Your taste profile used for future kits
- Influence the product roadmap
- 12 months of updates

### Tier 3: Producer Pack Service — Contact
- Custom kit generation pipeline for your brand
- Batch kit production
- White-label options
- Ongoing monthly kit delivery

## Payment Flow (Manual)

1. Producer emails or DMs interest
2. Send Stripe/PayPal invoice
3. On confirmation, generate kit using:
   ```
   cshot curate-pack "producer's description" --target 40
   cshot export-daw outputs/curated/kit --daw ableton --zipped
   ```
4. Deliver ZIP via email/Dropbox
5. Follow up for feedback:
   ```
   cshot feedback-pack outputs/curated/kit --name producer_name
   ```

## What the Beta Includes

- Direct line to the developer
- Your rated sounds train the model for better future kits
- Early access to Gradio web UI
- Priority feature requests
