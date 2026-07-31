<script setup>
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useLearningSession } from '../stores/useLearningSession'

const router = useRouter()
const session = useLearningSession()

onMounted(()=>{ if(!session.active) session.start() })
const current = computed(()=>session.current)

function next(){ session.next(); if(session.isFinished) router.push({name:'Stats'}) }
function open(){ if(current.value) router.push({name:current.value.routeName}) }
</script>

<template>
<div>
  <div v-if="current">
    <div>{{ current.title }}</div>
    <button @click="open">open</button>
    <button @click="next">next</button>
  </div>
</div>
</template>