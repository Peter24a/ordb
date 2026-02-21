"""Generate test_data/ folder with ~50 files for ordb integration testing."""
import os
import shutil
import struct
import io
from PIL import Image
from exif import Image as ExifImage
from mutagen.id3 import ID3, TIT2, TPE1, TALB
from mutagen.mp3 import MP3

OUT = os.path.join(os.path.dirname(__file__), "test_data")

def ensure_clean():
    if os.path.exists(OUT):
        shutil.rmtree(OUT)
    os.makedirs(OUT)

# --------------- helpers ---------------

def make_image(name, width=640, height=480, color="blue", fmt="JPEG"):
    img = Image.new("RGB", (width, height), color)
    path = os.path.join(OUT, name)
    img.save(path, fmt)
    return path

def make_image_with_exif(name, date_str, width=640, height=480, color="green"):
    """Create JPEG with EXIF DateTimeOriginal using exif library."""
    img = Image.new("RGB", (width, height), color)
    path = os.path.join(OUT, name)
    img.save(path, "JPEG")
    # Add EXIF with exif library (compatible with kamadak-exif)
    with open(path, 'rb') as f:
        exif_img = ExifImage(f)
    exif_img.datetime_original = date_str
    with open(path, 'wb') as f:
        f.write(exif_img.get_file())
    return path

def make_png(name, width=320, height=240, color="red"):
    img = Image.new("RGB", (width, height), color)
    path = os.path.join(OUT, name)
    img.save(path, "PNG")
    return path

def make_minimal_mp3(path):
    """Create a minimal valid MP3 file (silence frame)."""
    # MPEG1 Layer3 128kbps 44100Hz stereo frame header: FF FB 90 00
    frame_header = b'\xff\xfb\x90\x00'
    # A single MP3 frame is 417 bytes for 128kbps/44100Hz
    # frame_size = 144 * bitrate / sample_rate + padding = 144*128000/44100 = 417
    frame_data = frame_header + b'\x00' * 413
    with open(path, 'wb') as f:
        f.write(frame_data * 5)  # write a few frames for validity

def make_mp3_with_tags(name, artist, album, title):
    path = os.path.join(OUT, name)
    make_minimal_mp3(path)
    audio = MP3(path)
    audio.add_tags()
    audio.tags.add(TIT2(encoding=3, text=[title]))
    audio.tags.add(TPE1(encoding=3, text=[artist]))
    audio.tags.add(TALB(encoding=3, text=[album]))
    audio.save()
    return path

def make_mp3_without_tags(name):
    path = os.path.join(OUT, name)
    make_minimal_mp3(path)
    return path

def make_minimal_pdf(name, text="Test PDF content"):
    """Create a minimal valid PDF."""
    path = os.path.join(OUT, name)
    content = f"""%PDF-1.4
1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj
2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj
3 0 obj<</Type/Page/MediaBox[0 0 612 792]/Parent 2 0 R/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj
4 0 obj<</Length {len(text) + 44}>>stream
BT /F1 12 Tf 100 700 Td ({text}) Tj ET
endstream endobj
5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj
xref
0 6
trailer<</Size 6/Root 1 0 R>>
startxref
0
%%EOF"""
    with open(path, 'w') as f:
        f.write(content)
    return path

def make_zip(name):
    """Create a minimal valid ZIP (empty archive)."""
    path = os.path.join(OUT, name)
    # ZIP end of central directory record
    eocd = b'PK\x05\x06' + b'\x00' * 18
    with open(path, 'wb') as f:
        f.write(eocd)
    return path

def make_empty(name):
    path = os.path.join(OUT, name)
    with open(path, 'w') as f:
        pass
    return path

def make_garbage(name, size=1024):
    path = os.path.join(OUT, name)
    with open(path, 'wb') as f:
        f.write(os.urandom(size))
    return path

# --------------- main ---------------

def main():
    ensure_clean()
    count = 0

    # -- IMAGES WITH EXIF (landscapes, portraits, etc) --
    p = make_image_with_exif("foto_paisaje_2023.jpg", "2023:06:15 14:30:00", color="forestgreen")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - landscape with EXIF 2023")

    # Exact duplicate of the landscape
    dup = os.path.join(OUT, "foto_paisaje_COPIA.jpg")
    shutil.copy2(p, dup)
    count += 1
    print(f"  [{count}] foto_paisaje_COPIA.jpg - exact duplicate")

    p = make_image_with_exif("retrato_persona_2022.jpg", "2022:12:01 09:00:00", color="peachpuff")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - portrait with EXIF 2022")

    p = make_image_with_exif("grupo_familia_2024.jpg", "2024:03:20 18:45:00", color="lightyellow")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - group with EXIF 2024")

    p = make_image_with_exif("comida_cena_2023.jpg", "2023:11:25 20:00:00", color="orange")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - food with EXIF 2023")

    p = make_image_with_exif("edificio_2021.jpg", "2021:07:04 12:00:00", color="gray")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - architecture with EXIF 2021")

    p = make_image_with_exif("mascota_perro_2023.jpg", "2023:09:10 16:00:00", color="saddlebrown")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - pet with EXIF 2023")

    p = make_image_with_exif("auto_rojo_2022.jpg", "2022:05:18 11:30:00", color="red")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - vehicle with EXIF 2022")

    p = make_image_with_exif("documento_recibo_2023.jpg", "2023:01:15 08:00:00", color="white")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - document with EXIF 2023")

    # More EXIF images for variety (same month to test collision)
    p = make_image_with_exif("paisaje_playa_2023.jpg", "2023:06:20 10:00:00", color="skyblue")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - beach landscape, same month as first")

    p = make_image_with_exif("retrato_amigo_2022.jpg", "2022:12:15 14:00:00", color="salmon")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - second portrait, same month")

    # -- IMAGES WITHOUT EXIF (filesystem date) --
    p = make_png("foto_sin_exif_1.png", color="cyan")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - PNG without EXIF")

    p = make_png("foto_sin_exif_2.png", color="magenta")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - PNG without EXIF #2")

    p = make_image("foto_sin_exif_3.jpg", color="navy")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - JPEG without EXIF")

    p = make_png("captura_pantalla.png", color="darkgray")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - screenshot-like PNG")

    # -- SUSPICIOUS DATE (EXIF with 1970) --
    p = make_image_with_exif("foto_fecha_1970.jpg", "1970:01:01 00:00:00", color="black")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - EXIF with 1970 date (suspicious)")

    # -- DISGUISED FILE (.jpg renamed to .txt) --
    disguised_src = make_image("temp_disguised.jpg", color="purple")
    disguised_dst = os.path.join(OUT, "imagen_disfrazada.txt")
    os.rename(disguised_src, disguised_dst)
    count += 1
    print(f"  [{count}] imagen_disfrazada.txt - JPEG disguised as .txt")

    # -- CORRUPT / EMPTY FILES --
    p = make_empty("archivo_corrupto_vacio.jpg")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - 0 bytes (should be OMITIDO)")

    p = make_empty("archivo_vacio.png")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - 0 bytes PNG (should be OMITIDO)")

    p = make_garbage("basura_random.bin", 512)
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - random garbage bytes")

    p = make_garbage("basura_con_ext_jpg.jpg", 256)
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - garbage with .jpg extension")

    # -- MP3 WITH TAGS --
    p = make_mp3_with_tags("cancion_con_tags.mp3", "Los Fabulosos Cadillacs", "Vasos Vacios", "Matador")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - MP3 with artist/album")

    p = make_mp3_with_tags("rock_clasico.mp3", "Soda Stereo", "Cancion Animal", "De Musica Ligera")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - MP3 with tags #2")

    p = make_mp3_with_tags("cumbia_01.mp3", "Los Palmeras", "30 Anos", "La Vestida Celeste")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - MP3 with tags #3")

    p = make_mp3_with_tags("otro_rock.mp3", "Soda Stereo", "Dynamo", "Primavera 0")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - same artist, different album")

    # -- MP3 WITHOUT TAGS --
    p = make_mp3_without_tags("cancion_sin_tags.mp3")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - MP3 without any tags")

    p = make_mp3_without_tags("audio_desconocido.mp3")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - MP3 without tags #2")

    # -- PDFs --
    p = make_minimal_pdf("documento.pdf", "Factura de ejemplo")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - PDF document")

    p = make_minimal_pdf("reporte_anual.pdf", "Reporte financiero 2023")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - PDF report")

    p = make_minimal_pdf("contrato.pdf", "Contrato de servicios")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - PDF contract")

    # -- ZIPs --
    p = make_zip("archivo.zip")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - ZIP archive")

    p = make_zip("backup_fotos.zip")
    count += 1
    print(f"  [{count}] {os.path.basename(p)} - ZIP backup")

    # -- MORE IMAGES to reach ~50 --
    colors = [
        ("paisaje_montana.jpg", "2023:08:01 07:00:00", "darkgreen"),
        ("paisaje_rio.jpg", "2023:08:15 09:30:00", "teal"),
        ("selfie_playa.jpg", "2024:01:10 15:00:00", "tan"),
        ("gato_durmiendo.jpg", "2023:04:22 13:00:00", "dimgray"),
        ("perro_parque.jpg", "2023:05:30 17:00:00", "sienna"),
        ("pizza_casera.jpg", "2024:02:14 21:00:00", "tomato"),
        ("sushi_restaurant.jpg", "2023:10:05 19:30:00", "salmon"),
        ("moto_ruta.jpg", "2022:11:11 10:00:00", "darkred"),
        ("iglesia_colonial.jpg", "2021:09:28 11:00:00", "beige"),
        ("meme_gracioso.jpg", "2024:05:01 22:00:00", "yellow"),
        ("arte_digital.jpg", "2023:07:07 16:00:00", "violet"),
        ("grupo_amigos_asado.jpg", "2023:12:31 23:00:00", "coral"),
        ("bebe_primeros_pasos.jpg", "2024:06:15 10:30:00", "lavender"),
        ("atardecer_campo.jpg", "2023:03:21 18:45:00", "darkorange"),
        ("oficina_escritorio.jpg", "2022:08:20 09:00:00", "lightgray"),
        ("ticket_estacionamiento.jpg", "2023:02:28 14:00:00", "whitesmoke"),
        ("camion_ruta.jpg", "2022:04:10 06:00:00", "darkslategray"),
        ("jardin_flores.jpg", "2024:10:05 08:00:00", "pink"),
    ]

    for name, date, color in colors:
        p = make_image_with_exif(name, date, color=color)
        count += 1
        print(f"  [{count}] {os.path.basename(p)}")

    # One more duplicate (of a different file)
    src = os.path.join(OUT, "selfie_playa.jpg")
    dup2 = os.path.join(OUT, "selfie_playa_duplicado.jpg")
    shutil.copy2(src, dup2)
    count += 1
    print(f"  [{count}] selfie_playa_duplicado.jpg - second exact duplicate")

    print(f"\nTotal: {count} files created in {OUT}")

if __name__ == "__main__":
    main()
