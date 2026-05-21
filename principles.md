# Hava Durumu CLI - Temel Prensipler ve Değişmez Amacımız

Bu doküman, projenin sürümü ne olursa olsun (v1.0'dan v10.0'a ve ötesine kadar) asla değişmeyecek olan temel prensiplerimizi ve nihai amacımızı tanımlar. Projeye yapılan her katkı, eklenen her kod satırı ve her mimari değişiklik bu hedefler doğrultusunda değerlendirilmelidir.

---

## 🎯 Değişmez Amacımız (Hedefimiz)

Uygulamanın sürümü kaç olursa olsun, amacımız her zaman ve her koşulda şunları sağlamaktır:

1. **Daha Performanslı:** CPU ve kaynak kullanımı açısından en yüksek verimlilik.
2. **Daha Yüksek Uptime:** Kesintisiz hizmet, hata toleransı yüksek yapı ve her koşulda çalışabilirlik.
3. **Daha Gerçek & Doğru Veri:** Farklı kaynaklardan gelen verilerin en doğru ve tutarlı şekilde harmanlanması, hata payının sıfıra indirilmesi.
4. **Daha Hızlı Çalışma:** Milisaniyeler seviyesinde çalışma ve yanıt süreleri, optimize edilmiş ağ istekleri.
5. **0 Dependency (Sıfır Bağımlılık):** Üçüncü parti kütüphanelere bağımlılığın minimuma (mümkünse sıfıra) indirilmesi, hafif ve güvenli bir kod tabanı.
6. **Daha Az RAM Tüketimi:** Bellek yönetiminin optimize edilmesi, bellek sızıntılarının önlenmesi ve en düşük RAM ayak izi.
7. **Daha Çok Özellik:** Kullanıcı deneyimini zenginleştiren, performanstan ödün vermeyen yenilikçi ve faydalı özellikler.

---

## 🛡️ Prensiplerin Uygulanması

Geliştirilen her yeni özellik veya yapılan her refaktör işleminde kendimize şu soruları sormalıyız:
* *Bu değişiklik RAM veya CPU tüketimini artırıyor mu?*
* *Bağımsızlığı (Dependency) azaltmak veya tamamen kaldırmak için standart kütüphaneleri kullanabilir miyiz?*
* *Hata toleransını ve uptime süresini nasıl artırabiliriz?*
* *Veri doğruluğunu artırmak için consensus (ortak akıl) mekanizmasını nasıl daha iyi optimize edebiliriz?*
