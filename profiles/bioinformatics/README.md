# Bioinformatics Profile

Conversion profile for bioinformatics Perl codebases (BioPerl, etc).

## Common patterns:
- BioPerl (Bio::SeqIO, Bio::DB) → rust-bio / noodles
- FASTA/FASTQ parsing → noodles-fasta / noodles-fastq
- GFF/GTF parsing → noodles-gff
- SAM/BAM handling → noodles-sam / noodles-bam
- VCF processing → noodles-vcf
- Sequence alignment → rust-bio
- Phylogenetic trees → custom implementation
