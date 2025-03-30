<script setup lang="ts">
import { ref } from 'vue';

const base: Date = new Date();
const initialDate: string = new Date(base.getTime() - (base.getTimezoneOffset() * 60000)).toISOString();

const account = ref('');
const datetime = ref(initialDate.slice(0, 16));
const subject = ref('');
const amount = ref('');
const submitting = ref(false);

const handleSubmit = async () => {
  if (submitting.value) { return; }
  submitting.value = true;

  const payload = {
    account: account.value,
    datetime: new Date(datetime.value).toISOString(),
    subject: subject.value,
    amount: amount.value,
  };

  try {
    const response = await fetch('/api/submit', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(response.statusText);
    }

    alert('Submitted successfully.');
    subject.value = '';
    amount.value = '';
  } catch (error) {
    alert(error);
  } finally {
    submitting.value = false;
  }
};
</script>

<template>
  <div class="form-container">
    <form @submit.prevent="handleSubmit">
      <div class="form-group">
        <label for="account">Account</label>
        <select id="account" v-model="account" required>
          <option>Yuucho</option>
          <option>OCBC</option>
          <option>BCA</option>
          <option>Jenius</option>
          <option>Rakuten</option>
          <option>Wallet IDR</option>
          <option>Wallet JPY</option>
          <option>manaca</option>
        </select>
      </div>

      <div class="form-group">
        <label for="datetime">Datetime</label>
        <input id="datetime" v-model="datetime" type="datetime-local" required />
      </div>

      <div class="form-group">
        <label for="subject">Subject</label>
        <input id="subject" v-model="subject" type="text" />
      </div>

      <div class="form-group">
        <label for="amount">Amount</label>
        <input id="amount" v-model="amount" type="number" step="1" inputmode="numeric" required />
      </div>

      <button type="submit" :disabled="submitting">{{ !submitting ? "Submit" : "Sending..." }}</button>
    </form>
  </div>
</template>

<style scoped>
.form-container {
  min-width: 240px;
  margin: 2rem auto;
  padding: 1rem;
  border: 1px solid #ccc;
  border-radius: 8px;
  background-color: #f9f9f9;
}

h1 {
  text-align: center;
  margin-bottom: 1rem;
}

.form-group {
  margin-bottom: 1rem;
}

label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: bold;
}

input, select {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid #ccc;
  border-radius: 4px;
}

input[id="datetime"] {
  font-family: inherit;
}

button {
  display: block;
  width: 100%;
  padding: 0.75rem;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 1rem;
  cursor: pointer;
}

button:hover {
  background-color: #0056b3;
}

@media (prefers-color-scheme: dark) {
  .form-container {
    border-color: #444;
    background-color: #333;
    color: #f9f9f9;
  }

  input, select {
    background-color: #444;
    color: #f9f9f9;
    border-color: #555;
  }

  button {
    background-color: #0056b3;
  }

  button:hover {
    background-color: #003f8a;
  }
}
</style>
