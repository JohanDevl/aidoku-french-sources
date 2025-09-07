document.addEventListener("DOMContentLoaded", function () {
  loadSources();
});

async function loadSources() {
  const sourcesContainer = document.getElementById("sources-list");

  try {
    // Show loading state
    sourcesContainer.innerHTML =
      '<div class="loading">Loading sources...</div>';

    // Fetch sources data from local file
    const response = await fetch("./index.json");
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data = await response.json();
    const sources = data.sources || data; // Handle both new object format and legacy array format

    // Clear loading state
    sourcesContainer.innerHTML = "";

    // Define offline sources based on the README status
    const offlineSources = ["fr.reaperscans", "fr.mangascan", "fr.legacyscans", "fr.sushiscan"];

    // Define source types
    const sourceTypes = {
      "fr.astralmanga": "Madara",
      "fr.mangascantrad": "Madara",
      "fr.mangasorigines": "Madara",
      "fr.reaperscans": "Madara",
      "fr.lelmanga": "MangaThemesia",
      "fr.sushiscan": "MangaStream",
      "fr.sushiscans": "MangaStream",
      "fr.mangascan": "MMRCMS",
      "fr.animesama": "Custom",
      "fr.fmteam": "Custom",
      "fr.lelscanfr": "Custom",
      "fr.phenixscans": "Custom",
      "fr.poseidonscans": "Custom",
      "fr.legacyscans": "Custom",
    };

    // Sort sources: active first, then offline
    const sortedSources = sources.sort((a, b) => {
      const aOffline = offlineSources.includes(a.id);
      const bOffline = offlineSources.includes(b.id);

      if (aOffline === bOffline) {
        return a.name.localeCompare(b.name);
      }
      return aOffline - bOffline;
    });

    // Create source cards
    sortedSources.forEach((source, index) => {
      const sourceCard = createSourceCard(
        source,
        offlineSources,
        sourceTypes,
        index
      );
      sourcesContainer.appendChild(sourceCard);
    });

    // Update stats in the description
    updateStats(sources, offlineSources);
  } catch (error) {
    console.error("Error loading sources:", error);
    sourcesContainer.innerHTML =
      '<div class="loading">Failed to load sources. Please try refreshing the page.</div>';
  }
}

function createSourceCard(source, offlineSources, sourceTypes, index) {
  const isOffline = offlineSources.includes(source.id);
  const sourceType = sourceTypes[source.id] || "Unknown";

  const card = document.createElement("div");
  card.className = "source-card";
  card.style.setProperty("--index", index);

  const iconPath = source.iconURL || `./icons/${source.id}.png`;

  card.innerHTML = `
        <div class="source-header">
            <img src="${iconPath}" alt="${
    source.name
  }" class="source-icon" onerror="this.style.display='none'">
            <h3 class="source-name">${source.name}</h3>
        </div>
        
        <div class="source-status">
            <span class="badge ${isOffline ? "badge-offline" : "badge-active"}">
                ${isOffline ? "❌ Offline" : "✅ Active"}
            </span>
            <span class="badge badge-fr">FR</span>
            ${
              source.nsfw === 1
                ? '<span class="badge badge-nsfw">NSFW</span>'
                : ""
            }
        </div>
        
        <div class="source-version">
            Version ${source.version} · ${sourceType}
        </div>
        
        <div class="source-id">
            ${source.id}
        </div>
    `;

  return card;
}

function updateStats(sources, offlineSources) {
  const totalSources = sources.length;
  const activeSources = sources.filter(
    (source) => !offlineSources.includes(source.id)
  ).length;
  const offlineCount = sources.filter((source) =>
    offlineSources.includes(source.id)
  ).length;

  // Update the description paragraph
  const guideSection = document.querySelector(".guide-section > p:first-child");
  if (guideSection) {
    guideSection.textContent = `${totalSources} French sources for Aidoku (${activeSources} active, ${offlineCount} offline)`;
  }

  // Update page title
  document.title = `JohanDevl's French Sources - ${totalSources} Sources Available`;
}

// Add click handler for copying base URL
document.addEventListener("click", function (e) {
  if (e.target.classList.contains("base-url")) {
    navigator.clipboard
      .writeText(e.target.textContent)
      .then(() => {
        const originalText = e.target.textContent;
        e.target.textContent = "Copied to clipboard!";
        e.target.style.color = "#10b981";

        setTimeout(() => {
          e.target.textContent = originalText;
          e.target.style.color = "#10b981";
        }, 2000);
      })
      .catch((err) => {
        console.error("Failed to copy text: ", err);
      });
  }
});

// Add smooth scrolling for anchor links
document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
  anchor.addEventListener("click", function (e) {
    e.preventDefault();
    const target = document.querySelector(this.getAttribute("href"));
    if (target) {
      target.scrollIntoView({
        behavior: "smooth",
        block: "start",
      });
    }
  });
});

// Add intersection observer for animation on scroll
const observerOptions = {
  threshold: 0.1,
  rootMargin: "0px 0px -50px 0px",
};

const observer = new IntersectionObserver((entries) => {
  entries.forEach((entry) => {
    if (entry.isIntersecting) {
      entry.target.style.animationDelay = "0s";
      entry.target.classList.add("animate-in");
    }
  });
}, observerOptions);

// Observe all source cards when they're created
function observeCards() {
  document.querySelectorAll(".source-card").forEach((card) => {
    observer.observe(card);
  });
}

// Call observeCards after sources are loaded
setTimeout(observeCards, 1000);
