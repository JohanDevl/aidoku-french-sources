document.addEventListener("DOMContentLoaded", function () {
  loadSources();
});

async function loadSources() {
  const sourcesContainer = document.getElementById("sources-list");

  try {
    // Show loading state
    sourcesContainer.innerHTML =
      '<div class="loading">Loading sources...</div>';

    // Fetch sources data and offline count in parallel
    const [sourcesResponse, offlineCountResponse] = await Promise.all([
      fetch("./index.json"),
      fetch("./offline-count.json")
    ]);

    if (!sourcesResponse.ok) {
      throw new Error(`HTTP error! status: ${sourcesResponse.status}`);
    }

    const sourcesData = await sourcesResponse.json();
    const sources = sourcesData.sources || sourcesData; // Handle both new object format and legacy array format

    let offlineCount = 0;
    if (offlineCountResponse.ok) {
      const offlineData = await offlineCountResponse.json();
      offlineCount = offlineData.count || 0;
    }

    // Clear loading state
    sourcesContainer.innerHTML = "";

    // Sort sources alphabetically
    const sortedSources = sources.sort((a, b) => a.name.localeCompare(b.name));

    // Create source cards (all sources from index.json are active)
    sortedSources.forEach((source, index) => {
      const sourceCard = createSourceCard(
        source,
        index
      );
      sourcesContainer.appendChild(sourceCard);
    });

    // Update stats in the description
    updateStats(sources.length, offlineCount);
  } catch (error) {
    console.error("Error loading sources:", error);
    sourcesContainer.innerHTML =
      '<div class="loading">Failed to load sources. Please try refreshing the page.</div>';
  }
}

function createSourceCard(source, index) {
  const card = document.createElement("div");
  card.className = "source-card clickable";
  card.style.setProperty("--index", index);

  // Add click handler for download
  card.addEventListener('click', () => {
    if (source.downloadURL) {
      const link = document.createElement('a');
      link.href = source.downloadURL;
      link.download = source.downloadURL.split('/').pop();
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  });

  const iconPath = source.iconURL || `./icons/${source.id}.png`;

  card.innerHTML = `
        <div class="source-header">
            <img src="${iconPath}" alt="${
    source.name
  }" class="source-icon" onerror="this.style.display='none'">
            <h3 class="source-name">${source.name}</h3>
        </div>

        <div class="source-status">
            <span class="badge badge-active">
                ✅ Active
            </span>
            <span class="badge badge-fr">FR</span>
            ${
              source.contentRating === 2
                ? '<span class="badge badge-nsfw">NSFW</span>'
                : ""
            }
        </div>

        <div class="source-version">
            Version ${source.version}
        </div>

        <div class="source-id">
            ${source.id}
        </div>

        <div class="download-hint">
            Click to download
        </div>
    `;

  return card;
}

function updateStats(activeSources, offlineCount) {
  const totalSources = activeSources + offlineCount;

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
